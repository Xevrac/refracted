using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_ShowScoreDisplay
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.ShowScoreDisplay); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.ShowScoreDisplay)obj;
            //  Serialize ScoreStringId
            s.Write(value.ScoreStringId);
            //  Serialize array ScoreData
            Rts.Serialization.Reference.Write(s, value.ScoreData, () =>
            {
                s.WriteVarInt32(value.ScoreData.Length);
                for(int i = 0 ; i < value.ScoreData.Length ; ++i)
                {
                    s.Write(value.ScoreData[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.ShowScoreDisplay)) as Rts.CnC.Messages.Client.ShowScoreDisplay;
            //  Deserialize ScoreStringId
            s.Read(out value.ScoreStringId);
            //  Deserialize array ScoreData
            Rts.Serialization.Reference.Read(s, out value.ScoreData, () =>
            {
                int length = s.ReadVarInt32();
                System.Int32[] tmp = new System.Int32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
