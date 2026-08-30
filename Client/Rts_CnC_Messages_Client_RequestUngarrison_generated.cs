using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestUngarrison
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestUngarrison); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestUngarrison)obj;
            //  Serialize GarrisonPlayerId
            s.Write(value.GarrisonPlayerId);
            //  Serialize GarrisonId
            s.Write(value.GarrisonId);
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize array UnitIds
            Rts.Serialization.Reference.Write(s, value.UnitIds, () =>
            {
                s.WriteVarInt32(value.UnitIds.Length);
                for(int i = 0 ; i < value.UnitIds.Length ; ++i)
                {
                    s.Write(value.UnitIds[i]);
                }
            });
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestUngarrison)) as Rts.CnC.Messages.Client.RequestUngarrison;
            //  Deserialize GarrisonPlayerId
            s.Read(out value.GarrisonPlayerId);
            //  Deserialize GarrisonId
            s.Read(out value.GarrisonId);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize array UnitIds
            Rts.Serialization.Reference.Read(s, out value.UnitIds, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
