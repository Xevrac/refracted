using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AutomatedTestWaypoints
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AutomatedTestWaypoints); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AutomatedTestWaypoints)obj;
            //  Serialize array WaypointData
            Rts.Serialization.Reference.Write(s, value.WaypointData, () =>
            {
                s.WriteVarInt32(value.WaypointData.Length);
                for(int i = 0 ; i < value.WaypointData.Length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AutomatedTestWaypoints_Element.Serializer.Serialize(s, value.WaypointData[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AutomatedTestWaypoints)) as Rts.CnC.Messages.Client.AutomatedTestWaypoints;
            //  Deserialize array WaypointData
            Rts.Serialization.Reference.Read(s, out value.WaypointData, () =>
            {
                int length = s.ReadVarInt32();
                Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element[] tmp = new Rts.CnC.Messages.Client.AutomatedTestWaypoints.Element[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AutomatedTestWaypoints_Element.Serializer.DeserializeValue(s, ref tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
