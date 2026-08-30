using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemoveEntityHighlights
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RemoveEntityHighlights); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RemoveEntityHighlights)obj;
            //  Serialize array Ids
            Rts.Serialization.Reference.Write(s, value.Ids, () =>
            {
                s.WriteVarInt32(value.Ids.Length);
                for(int i = 0 ; i < value.Ids.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemoveEntityHighlights_Element.Serializer.Serialize(s, value.Ids[i]);
                }
            });
            //  Serialize Objective
            s.Write(value.Objective);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RemoveEntityHighlights)) as Rts.CnC.Messages.Client.RemoveEntityHighlights;
            //  Deserialize array Ids
            Rts.Serialization.Reference.Read(s, out value.Ids, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.Client.RemoveEntityHighlights.Element[] tmp = new Rts.CnC.Messages.Client.RemoveEntityHighlights.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemoveEntityHighlights_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });
            //  Deserialize Objective
            s.Read(out value.Objective);

            return value;
        }
        
    }
}
