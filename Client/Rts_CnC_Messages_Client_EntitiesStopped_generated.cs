using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntitiesStopped
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntitiesStopped); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntitiesStopped)obj;
            //  Serialize TimeStamp
            s.Write(value.TimeStamp);
            //  Serialize array EntityData
            Rts.Serialization.Reference.Write(s, value.EntityData, () =>
            {
                s.WriteVarInt32(value.EntityData.Length);
                for(int i = 0 ; i < value.EntityData.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntitiesStopped_Element.Serializer.Serialize(s, value.EntityData[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntitiesStopped)) as Rts.CnC.Messages.Client.EntitiesStopped;
            //  Deserialize TimeStamp
            s.Read(out value.TimeStamp);
            //  Deserialize array EntityData
            Rts.Serialization.Reference.Read(s, out value.EntityData, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.Client.EntitiesStopped.Element[] tmp = new Rts.CnC.Messages.Client.EntitiesStopped.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntitiesStopped_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
